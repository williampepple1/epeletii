import React from "react";
import { useGameStore } from "../store/gameStore";

export const LeaderboardModal: React.FC = () => {
  const leaderboard = useGameStore((s) => s.leaderboard);
  const leaderboardOpen = useGameStore((s) => s.leaderboardOpen);
  const setLeaderboardOpen = useGameStore((s) => s.setLeaderboardOpen);

  if (!leaderboardOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-50 p-4">
      <div className="bg-[var(--surface)] w-full max-w-lg rounded-2xl shadow-2xl p-6 relative flex flex-col max-h-[85vh] border border-stone-200 dark:border-stone-850">
        {/* Header */}
        <div className="flex items-center justify-between pb-4 border-b border-stone-150 dark:border-stone-800">
          <h3 className="text-xl font-bold flex items-center gap-2 text-[var(--foreground)]">
            <span>🏆</span> Leaderboard
          </h3>
          <button
            onClick={() => setLeaderboardOpen(false)}
            className="text-stone-400 hover:text-stone-600 dark:hover:text-stone-200 transition-colors text-2xl"
          >
            &times;
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto py-4 space-y-3">
          {leaderboard.length === 0 ? (
            <p className="text-center text-stone-550 py-8">No stats recorded yet. Play a game to rank up!</p>
          ) : (
            <div className="min-w-full inline-block align-middle">
              <div className="overflow-hidden border border-stone-200 dark:border-stone-800 rounded-xl">
                <table className="min-w-full divide-y divide-stone-200 dark:divide-stone-800">
                  <thead className="bg-stone-50 dark:bg-stone-850">
                    <tr>
                      <th scope="col" className="px-4 py-3 text-left text-xs font-bold text-stone-500 uppercase tracking-wider">Rank</th>
                      <th scope="col" className="px-4 py-3 text-left text-xs font-bold text-stone-500 uppercase tracking-wider">Player</th>
                      <th scope="col" className="px-4 py-3 text-center text-xs font-bold text-stone-500 uppercase tracking-wider">Played</th>
                      <th scope="col" className="px-4 py-3 text-center text-xs font-bold text-stone-500 uppercase tracking-wider">Won</th>
                      <th scope="col" className="px-4 py-3 text-right text-xs font-bold text-stone-500 uppercase tracking-wider">Total Score</th>
                    </tr>
                  </thead>
                  <tbody className="bg-white dark:bg-stone-900 divide-y divide-stone-200 dark:divide-stone-800">
                    {leaderboard.map((entry, index) => {
                      const isTop3 = index < 3;
                      const badgeColor = index === 0 ? "bg-amber-100 text-amber-700" : index === 1 ? "bg-stone-100 text-stone-700" : "bg-orange-100 text-orange-700";
                      
                      return (
                        <tr key={index} className="hover:bg-stone-50/50 dark:hover:bg-stone-850/30 transition-colors">
                          <td className="px-4 py-3.5 whitespace-nowrap text-sm font-semibold">
                            {isTop3 ? (
                              <span className={`inline-flex items-center justify-center w-6 h-6 rounded-full text-xs font-bold ${badgeColor}`}>
                                {index + 1}
                              </span>
                            ) : (
                              <span className="text-stone-500 dark:text-stone-450 pl-2">{index + 1}</span>
                            )}
                          </td>
                          <td className="px-4 py-3.5 whitespace-nowrap text-sm font-bold text-stone-900 dark:text-stone-100">
                            {entry.display_name}
                          </td>
                          <td className="px-4 py-3.5 whitespace-nowrap text-sm text-center text-stone-600 dark:text-stone-400">
                            {entry.games_played}
                          </td>
                          <td className="px-4 py-3.5 whitespace-nowrap text-sm text-center text-stone-600 dark:text-stone-400">
                            {entry.games_won}
                          </td>
                          <td className="px-4 py-3.5 whitespace-nowrap text-sm font-black text-right text-amber-600 dark:text-amber-400">
                            {entry.total_score}
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="pt-4 border-t border-stone-150 dark:border-stone-800 flex justify-end">
          <button
            onClick={() => setLeaderboardOpen(false)}
            className="px-5 py-2 bg-stone-150 hover:bg-stone-200 dark:bg-stone-800 dark:hover:bg-stone-750 text-stone-850 dark:text-stone-200 font-semibold rounded-xl text-sm transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};
